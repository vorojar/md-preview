package app.mdpreview.mobile;

import android.app.Activity;
import android.content.ActivityNotFoundException;
import android.content.ContentResolver;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.database.Cursor;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.os.Parcelable;
import android.provider.OpenableColumns;
import android.provider.Settings;
import android.webkit.ValueCallback;
import android.webkit.JavascriptInterface;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.webkit.WebResourceRequest;

import org.json.JSONException;
import org.json.JSONObject;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.CharBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CharsetDecoder;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;

public final class MainActivity extends Activity {
    private static final int OPEN_DOCUMENT_REQUEST = 7;
    private WebView webView;
    private Uri pendingUri;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        webView = new WebView(this);
        setContentView(webView);
        configureWebView();
        pendingUri = uriFromIntent(getIntent());
        webView.loadUrl("file:///android_asset/preview.html");
    }

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
        Uri uri = uriFromIntent(intent);
        if (uri != null) {
            openUri(uri);
        }
    }

    @Override
    @SuppressWarnings("deprecation")
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode == OPEN_DOCUMENT_REQUEST && resultCode == RESULT_OK && data != null) {
            Uri uri = data.getData();
            if (uri != null) {
                persistReadPermission(uri, data.getFlags());
                openUri(uri);
            }
        }
    }

    @SuppressWarnings("deprecation")
    private void configureWebView() {
        WebSettings settings = webView.getSettings();
        settings.setJavaScriptEnabled(true);
        settings.setDomStorageEnabled(false);
        settings.setAllowFileAccess(true);
        settings.setAllowContentAccess(true);
        settings.setAllowFileAccessFromFileURLs(false);
        settings.setAllowUniversalAccessFromFileURLs(false);
        settings.setBlockNetworkLoads(true);

        webView.addJavascriptInterface(new Bridge(), "MDPreviewAndroid");
        webView.setWebViewClient(new WebViewClient() {
            @Override
            public void onPageFinished(WebView view, String url) {
                if (pendingUri != null) {
                    Uri uri = pendingUri;
                    pendingUri = null;
                    openUri(uri);
                }
            }

            @Override
            public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest request) {
                return handleNavigation(request.getUrl());
            }

            @Override
            @SuppressWarnings("deprecation")
            public boolean shouldOverrideUrlLoading(WebView view, String url) {
                return handleNavigation(Uri.parse(url));
            }
        });
    }

    @SuppressWarnings("deprecation")
    private Uri uriFromIntent(Intent intent) {
        if (intent == null) {
            return null;
        }
        if (Intent.ACTION_VIEW.equals(intent.getAction()) && intent.getData() != null) {
            return intent.getData();
        }
        if (Intent.ACTION_SEND.equals(intent.getAction())) {
            Parcelable stream;
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                stream = intent.getParcelableExtra(Intent.EXTRA_STREAM, Parcelable.class);
            } else {
                stream = intent.getParcelableExtra(Intent.EXTRA_STREAM);
            }
            if (stream instanceof Uri) {
                return (Uri) stream;
            }
        }
        if (Intent.ACTION_SEND_MULTIPLE.equals(intent.getAction())) {
            ArrayList<Uri> streams;
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                streams = intent.getParcelableArrayListExtra(Intent.EXTRA_STREAM, Uri.class);
            } else {
                streams = intent.getParcelableArrayListExtra(Intent.EXTRA_STREAM);
            }
            if (streams != null && !streams.isEmpty()) {
                return streams.get(0);
            }
        }
        return null;
    }

    private void openUri(Uri uri) {
        try {
            String markdown = readText(uri);
            JSONObject payload = new JSONObject();
            payload.put("markdown", markdown);
            payload.put("name", displayName(uri));
            payload.put("baseHref", "file".equals(uri.getScheme()) ? baseHref(uri) : "");
            evaluate("window.MDPreview && window.MDPreview.render(" + payload + ");");
        } catch (IOException | JSONException e) {
            JSONObject payload = new JSONObject();
            try {
                payload.put("markdown", "Cannot read " + displayName(uri));
                payload.put("name", "Read error.md");
                payload.put("baseHref", "");
            } catch (JSONException ignored) {
            }
            evaluate("window.MDPreview && window.MDPreview.render(" + payload + ");");
        }
    }

    private String readText(Uri uri) throws IOException {
        ContentResolver resolver = getContentResolver();
        try (InputStream input = resolver.openInputStream(uri)) {
            if (input == null) {
                throw new IOException("Cannot open input stream");
            }
            ByteArrayOutputStream output = new ByteArrayOutputStream();
            byte[] buffer = new byte[8192];
            int read;
            while ((read = input.read(buffer)) != -1) {
                output.write(buffer, 0, read);
            }
            return decodeMarkdown(output.toByteArray());
        }
    }

    private String decodeMarkdown(byte[] bytes) {
        if (bytes.length >= 3
            && (bytes[0] & 0xff) == 0xef
            && (bytes[1] & 0xff) == 0xbb
            && (bytes[2] & 0xff) == 0xbf) {
            return new String(bytes, 3, bytes.length - 3, StandardCharsets.UTF_8);
        }
        if (bytes.length >= 2 && (bytes[0] & 0xff) == 0xff && (bytes[1] & 0xff) == 0xfe) {
            return StandardCharsets.UTF_16LE.decode(ByteBuffer.wrap(bytes, 2, bytes.length - 2)).toString();
        }
        if (bytes.length >= 2 && (bytes[0] & 0xff) == 0xfe && (bytes[1] & 0xff) == 0xff) {
            return StandardCharsets.UTF_16BE.decode(ByteBuffer.wrap(bytes, 2, bytes.length - 2)).toString();
        }

        CharsetDecoder decoder = StandardCharsets.UTF_8.newDecoder()
            .onMalformedInput(CodingErrorAction.REPLACE)
            .onUnmappableCharacter(CodingErrorAction.REPLACE);
        try {
            CharBuffer chars = decoder.decode(ByteBuffer.wrap(bytes).order(ByteOrder.BIG_ENDIAN));
            return chars.toString();
        } catch (CharacterCodingException ignored) {
            return new String(bytes, StandardCharsets.UTF_8);
        }
    }

    private String displayName(Uri uri) {
        if ("content".equals(uri.getScheme())) {
            try (Cursor cursor = getContentResolver().query(uri, null, null, null, null)) {
                if (cursor != null && cursor.moveToFirst()) {
                    int index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME);
                    if (index >= 0) {
                        String name = cursor.getString(index);
                        if (name != null && !name.isEmpty()) {
                            return name;
                        }
                    }
                }
            }
        }
        String path = uri.getLastPathSegment();
        if (path == null || path.isEmpty()) {
            return "Untitled.md";
        }
        int slash = path.lastIndexOf('/');
        return slash >= 0 ? path.substring(slash + 1) : path;
    }

    private String baseHref(Uri uri) {
        String text = uri.toString();
        int slash = text.lastIndexOf('/');
        return slash >= 0 ? text.substring(0, slash + 1) : "";
    }

    @SuppressWarnings("deprecation")
    private void openDocumentPicker() {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        intent.setType("*/*");
        intent.putExtra(Intent.EXTRA_MIME_TYPES, new String[] {
            "text/markdown",
            "text/x-markdown",
            "text/plain",
            "application/octet-stream"
        });
        intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION);
        try {
            startActivityForResult(intent, OPEN_DOCUMENT_REQUEST);
        } catch (ActivityNotFoundException ignored) {
        }
    }

    private void persistReadPermission(Uri uri, int flags) {
        int readFlag = Intent.FLAG_GRANT_READ_URI_PERMISSION;
        int persistableFlag = Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION;
        if ((flags & readFlag) == 0 || (flags & persistableFlag) == 0) {
            return;
        }
        try {
            getContentResolver().takePersistableUriPermission(uri, readFlag);
        } catch (SecurityException ignored) {
        }
    }

    private void evaluate(String script) {
        webView.evaluateJavascript(script, (ValueCallback<String>) null);
    }

    private void openExternalUrl(String url) {
        Uri uri = Uri.parse(url);
        openExternalUri(uri);
    }

    private boolean handleNavigation(Uri uri) {
        String scheme = uri.getScheme();
        if (scheme == null || scheme.equalsIgnoreCase("file")) {
            return false;
        }
        if (scheme.equalsIgnoreCase("javascript")
            || scheme.equalsIgnoreCase("data")
            || scheme.equalsIgnoreCase("vbscript")) {
            return true;
        }
        if (scheme.equalsIgnoreCase("http")
            || scheme.equalsIgnoreCase("https")
            || scheme.equalsIgnoreCase("mailto")) {
            openExternalUri(uri);
            return true;
        }
        return true;
    }

    private void openExternalUri(Uri uri) {
        String scheme = uri.getScheme();
        if (scheme == null
            || (!scheme.equalsIgnoreCase("http")
            && !scheme.equalsIgnoreCase("https")
            && !scheme.equalsIgnoreCase("mailto"))) {
            return;
        }
        Intent intent = new Intent(Intent.ACTION_VIEW, uri);
        try {
            startActivity(intent);
        } catch (ActivityNotFoundException ignored) {
        }
    }

    private void openDefaultSettings() {
        String wpsPackage = installedWpsPackage();
        if (wpsPackage != null && openAppSettings(wpsPackage)) {
            return;
        }

        Intent defaultsIntent = new Intent(Settings.ACTION_MANAGE_DEFAULT_APPS_SETTINGS);
        try {
            startActivity(defaultsIntent);
            return;
        } catch (ActivityNotFoundException ignored) {
        }

        if (openAppSettings(getPackageName())) {
            return;
        }

        try {
            startActivity(new Intent(Settings.ACTION_SETTINGS));
        } catch (ActivityNotFoundException ignored) {
        }
    }

    private boolean openAppSettings(String packageName) {
        Intent intent = new Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS);
        intent.setData(Uri.parse("package:" + packageName));
        try {
            startActivity(intent);
            return true;
        } catch (ActivityNotFoundException ignored) {
            return false;
        }
    }

    private String installedWpsPackage() {
        String[] candidates = {
            "cn.wps.moffice_eng",
            "cn.wps.moffice_i18n",
            "cn.wps.moffice",
            "com.kingsoft.moffice_pro"
        };
        PackageManager packageManager = getPackageManager();
        for (String candidate : candidates) {
            try {
                packageManager.getPackageInfo(candidate, 0);
                return candidate;
            } catch (PackageManager.NameNotFoundException ignored) {
            }
        }
        return null;
    }

    private final class Bridge {
        @JavascriptInterface
        public void openFile() {
            runOnUiThread(MainActivity.this::openDocumentPicker);
        }

        @JavascriptInterface
        public void openExternal(String url) {
            runOnUiThread(() -> openExternalUrl(url));
        }

        @JavascriptInterface
        public void openDefaultSettings() {
            runOnUiThread(MainActivity.this::openDefaultSettings);
        }
    }
}
