import UniformTypeIdentifiers
import UIKit
import WebKit

final class PreviewViewController: UIViewController {
    private struct RecentDocument: Codable {
        let name: String
        let fileName: String
    }

    private struct RecentPayload: Encodable {
        let id: String
        let name: String
    }

    private static let recentKey = "recentDocuments"

    private lazy var webView: WKWebView = {
        let configuration = WKWebViewConfiguration()
        configuration.userContentController.add(self, name: "mdPreview")
        let view = WKWebView(frame: .zero, configuration: configuration)
        view.isOpaque = false
        view.backgroundColor = .systemBackground
        view.scrollView.backgroundColor = .systemBackground
        view.scrollView.contentInsetAdjustmentBehavior = .never
        return view
    }()

    private var pendingURL: URL?

    override func loadView() {
        view = webView
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        loadPreviewShell()
    }

    func openDocument(at url: URL) {
        if webView.url == nil {
            pendingURL = url
            return
        }
        renderDocument(at: url)
    }

    private func loadPreviewShell() {
        guard let htmlURL = Bundle.main.url(forResource: "preview", withExtension: "html", subdirectory: "shared"),
              let readAccessURL = Bundle.main.resourceURL?.appendingPathComponent("shared") else {
            return
        }
        webView.navigationDelegate = self
        webView.loadFileURL(htmlURL, allowingReadAccessTo: readAccessURL)
    }

    private func showDocumentPicker() {
        let types: [UTType] = [
            UTType(filenameExtension: "md") ?? .plainText,
            UTType(filenameExtension: "markdown") ?? .plainText,
            .plainText
        ]
        let picker = UIDocumentPickerViewController(forOpeningContentTypes: types, asCopy: false)
        picker.delegate = self
        picker.allowsMultipleSelection = false
        present(picker, animated: true)
    }

    private func renderDocument(at url: URL, displayName preferredName: String? = nil, shouldSaveRecent: Bool = true) {
        let documentName: String
        if let preferredName, !preferredName.isEmpty {
            documentName = preferredName
        } else {
            documentName = url.lastPathComponent.isEmpty ? "Untitled.md" : url.lastPathComponent
        }

        let didStartAccessing = url.startAccessingSecurityScopedResource()
        defer {
            if didStartAccessing {
                url.stopAccessingSecurityScopedResource()
            }
        }

        do {
            let data = try Data(contentsOf: url)
            if shouldSaveRecent {
                saveRecent(data: data, name: documentName)
            }
            let markdown = decodeMarkdown(data)
            let baseHref = url.isFileURL ? url.deletingLastPathComponent().absoluteString : ""
            let payload = PreviewPayload(
                markdown: markdown,
                name: documentName,
                baseHref: baseHref
            )
            let encoded = try JSONEncoder().encode(payload)
            let json = String(decoding: encoded, as: UTF8.self)
            webView.evaluateJavaScript("window.MDPreview && window.MDPreview.render(\(json));")
        } catch {
            let message = "Cannot read \(documentName)"
            webView.evaluateJavaScript("window.MDPreview && window.MDPreview.render({markdown:\(message.jsStringLiteral),name:'Read error.md',baseHref:''});")
        }
    }

    private func printDocument() {
        let controller = UIPrintInteractionController.shared
        let info = UIPrintInfo(dictionary: nil)
        info.outputType = .general
        info.jobName = webView.title?.replacingOccurrences(of: " - MD Preview", with: "") ?? "MD Preview"
        controller.printInfo = info
        let formatter = webView.viewPrintFormatter()
        formatter.perPageContentInsets = UIEdgeInsets(top: 36, left: 36, bottom: 36, right: 36)
        controller.printFormatter = formatter
        controller.present(animated: true)
    }

    private func recentDocuments() -> [RecentDocument] {
        guard let data = UserDefaults.standard.data(forKey: Self.recentKey),
              let items = try? JSONDecoder().decode([RecentDocument].self, from: data) else {
            return []
        }
        return items
    }

    private func recentDirectory() -> URL? {
        guard let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else {
            return nil
        }
        let directory = support.appendingPathComponent("RecentDocuments", isDirectory: true)
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        return directory
    }

    private func saveRecent(data: Data, name: String) {
        guard let directory = recentDirectory() else {
            return
        }
        let displayName = Self.cleanRecentName(name)
        let safeName = displayName.replacingOccurrences(of: "/", with: "-")
        let fileName = UUID().uuidString + "-" + safeName
        let fileURL = directory.appendingPathComponent(fileName)
        do {
            try data.write(to: fileURL, options: .atomic)
        } catch {
            return
        }
        var items = recentDocuments()
        let removed = items.filter { Self.cleanRecentName($0.name) == displayName }
        removed.forEach { item in
            try? FileManager.default.removeItem(at: directory.appendingPathComponent(item.fileName))
        }
        items.removeAll { Self.cleanRecentName($0.name) == displayName }
        items.insert(RecentDocument(name: displayName, fileName: fileName), at: 0)
        let trimmed = Array(items.prefix(8))
        items.dropFirst(8).forEach { item in
            try? FileManager.default.removeItem(at: directory.appendingPathComponent(item.fileName))
        }
        if let data = try? JSONEncoder().encode(trimmed) {
            UserDefaults.standard.set(data, forKey: Self.recentKey)
        }
        sendRecentToWeb()
    }

    private func sendRecentToWeb() {
        var seenNames = Set<String>()
        let payload = recentDocuments().enumerated().compactMap { index, item -> RecentPayload? in
            let name = Self.cleanRecentName(item.name)
            guard !seenNames.contains(name) else {
                return nil
            }
            seenNames.insert(name)
            return RecentPayload(id: String(index), name: name)
        }
        guard let data = try? JSONEncoder().encode(payload) else {
            return
        }
        let json = String(decoding: data, as: UTF8.self)
        webView.evaluateJavaScript("window.MDPreview && window.MDPreview.setRecent(\(json));")
    }

    private func openRecent(id: String) {
        guard let index = Int(id) else {
            return
        }
        let items = recentDocuments()
        guard items.indices.contains(index) else {
            return
        }
        guard let directory = recentDirectory() else {
            return
        }
        renderDocument(
            at: directory.appendingPathComponent(items[index].fileName),
            displayName: Self.cleanRecentName(items[index].name),
            shouldSaveRecent: false
        )
    }

    private static func cleanRecentName(_ name: String) -> String {
        let fallback = name.isEmpty ? "Untitled.md" : name
        let uuidLength = 36
        guard name.count > uuidLength + 1 else {
            return fallback
        }
        let separator = name.index(name.startIndex, offsetBy: uuidLength)
        guard name[separator] == "-" else {
            return fallback
        }
        let prefix = String(name[..<separator])
        guard UUID(uuidString: prefix) != nil else {
            return fallback
        }
        let rest = String(name[name.index(after: separator)...])
        return rest.isEmpty ? fallback : rest
    }
}

extension PreviewViewController: WKNavigationDelegate {
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        sendRecentToWeb()
        if let pendingURL {
            self.pendingURL = nil
            renderDocument(at: pendingURL)
        }
    }

    func webView(
        _ webView: WKWebView,
        decidePolicyFor navigationAction: WKNavigationAction,
        decisionHandler: @escaping @MainActor @Sendable (WKNavigationActionPolicy) -> Void
    ) {
        guard navigationAction.navigationType == .linkActivated,
              let url = navigationAction.request.url else {
            decisionHandler(.allow)
            return
        }
        if ["javascript", "data", "vbscript"].contains(url.scheme?.lowercased() ?? "") {
            decisionHandler(.cancel)
            return
        }
        guard ["http", "https", "mailto"].contains(url.scheme?.lowercased() ?? "") else {
            decisionHandler(.allow)
            return
        }
        UIApplication.shared.open(url)
        decisionHandler(.cancel)
    }
}

extension PreviewViewController: WKScriptMessageHandler {
    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        guard message.name == "mdPreview" else {
            return
        }
        if let body = message.body as? [String: Any],
           let action = body["action"] as? String {
            if action == "open" {
                showDocumentPicker()
            } else if action == "print" {
                printDocument()
            } else if action == "recent" {
                sendRecentToWeb()
            } else if action == "openRecent",
                      let id = body["id"] as? String {
                openRecent(id: id)
            } else if action == "openExternal",
                      let urlString = body["url"] as? String,
                      let url = URL(string: urlString),
                      ["http", "https", "mailto"].contains(url.scheme?.lowercased() ?? "") {
                UIApplication.shared.open(url)
            }
            return
        }
        if let action = message.body as? String, action == "open" {
            showDocumentPicker()
        }
    }
}

extension PreviewViewController: UIDocumentPickerDelegate {
    func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) {
        guard let url = urls.first else {
            return
        }
        openDocument(at: url)
    }
}

private struct PreviewPayload: Encodable {
    let markdown: String
    let name: String
    let baseHref: String
}

private func decodeMarkdown(_ data: Data) -> String {
    if data.starts(with: [0xEF, 0xBB, 0xBF]) {
        return String(decoding: data.dropFirst(3), as: UTF8.self)
    }
    if data.starts(with: [0xFF, 0xFE]),
       let text = String(data: data.dropFirst(2), encoding: .utf16LittleEndian) {
        return text
    }
    if data.starts(with: [0xFE, 0xFF]),
       let text = String(data: data.dropFirst(2), encoding: .utf16BigEndian) {
        return text
    }
    return String(decoding: data, as: UTF8.self)
}

private extension String {
    var jsStringLiteral: String {
        guard let data = try? JSONEncoder().encode(self) else {
            return "\"\""
        }
        return String(decoding: data, as: UTF8.self)
    }
}
