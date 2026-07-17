import Cocoa
import FinderSync

@objc(MDPreviewFinderSync)
final class MDPreviewFinderSync: FIFinderSync {
    override init() {
        super.init()
        var directories = [
            URL(fileURLWithPath: NSHomeDirectory()),
            URL(fileURLWithPath: "/Users"),
            URL(fileURLWithPath: "/Volumes"),
            URL(fileURLWithPath: "/")
        ]
        directories.append(contentsOf: FileManager.default.mountedVolumeURLs(
            includingResourceValuesForKeys: nil,
            options: []
        ) ?? [])
        FIFinderSyncController.default().directoryURLs = Set(directories)
    }

    override func menu(for menuKind: FIMenuKind) -> NSMenu? {
        guard menuKind == .contextualMenuForContainer || menuKind == .contextualMenuForItems else {
            return nil
        }
        let menu = NSMenu(title: "")
        menu.addItem(newFileSubmenu())
        menu.addItem(actionItem(title: "复制路径", action: #selector(copyPath(_:))))
        menu.addItem(actionItem(title: "在终端打开", action: #selector(openInTerminal(_:))))
        return menu
    }

    @objc private func createMarkdown(_ sender: NSMenuItem) { send(action: "create", kind: "md") }
    @objc private func createText(_ sender: NSMenuItem) { send(action: "create", kind: "txt") }
    @objc private func createJSON(_ sender: NSMenuItem) { send(action: "create", kind: "json") }
    @objc private func createHTML(_ sender: NSMenuItem) { send(action: "create", kind: "html") }
    @objc private func openInTerminal(_ sender: NSMenuItem) { send(action: "terminal") }

    @objc private func copyPath(_ sender: NSMenuItem) {
        let controller = FIFinderSyncController.default()
        let paths: [String]
        if let selected = controller.selectedItemURLs(), !selected.isEmpty {
            paths = selected.map(\.path)
        } else if let targeted = controller.targetedURL() {
            paths = [targeted.path]
        } else {
            NSSound.beep()
            return
        }
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(paths.joined(separator: "\n"), forType: .string)
    }

    private func send(action: String, kind: String? = nil) {
        guard let folder = targetFolderURL() else {
            NSSound.beep()
            return
        }
        var components = URLComponents()
        components.scheme = "mdpreview"
        components.host = "finder"
        components.queryItems = [
            URLQueryItem(name: "action", value: action),
            URLQueryItem(name: "path", value: folder.path)
        ]
        if let kind {
            components.queryItems?.append(URLQueryItem(name: "kind", value: kind))
        }
        guard let url = components.url, NSWorkspace.shared.open(url) else {
            NSSound.beep()
            return
        }
    }

    private func targetFolderURL() -> URL? {
        let controller = FIFinderSyncController.default()
        if let selected = controller.selectedItemURLs()?.first {
            var isDirectory = ObjCBool(false)
            if FileManager.default.fileExists(atPath: selected.path, isDirectory: &isDirectory) {
                return isDirectory.boolValue ? selected : selected.deletingLastPathComponent()
            }
        }
        if let targeted = controller.targetedURL() {
            var isDirectory = ObjCBool(false)
            if FileManager.default.fileExists(atPath: targeted.path, isDirectory: &isDirectory) {
                return isDirectory.boolValue ? targeted : targeted.deletingLastPathComponent()
            }
        }
        return nil
    }

    private func newFileSubmenu() -> NSMenuItem {
        let item = NSMenuItem(title: "新建文件", action: nil, keyEquivalent: "")
        let submenu = NSMenu(title: "新建文件")
        submenu.addItem(actionItem(title: "Markdown (.md)", action: #selector(createMarkdown(_:))))
        submenu.addItem(actionItem(title: "Text (.txt)", action: #selector(createText(_:))))
        submenu.addItem(actionItem(title: "JSON (.json)", action: #selector(createJSON(_:))))
        submenu.addItem(actionItem(title: "HTML (.html)", action: #selector(createHTML(_:))))
        item.submenu = submenu
        return item
    }

    private func actionItem(title: String, action: Selector) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: action, keyEquivalent: "")
        item.target = self
        return item
    }
}
