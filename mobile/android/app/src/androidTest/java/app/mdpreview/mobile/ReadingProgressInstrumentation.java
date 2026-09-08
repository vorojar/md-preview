package app.mdpreview.mobile;

import android.app.Activity;
import android.app.Instrumentation;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.webkit.WebView;
import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;

/** Exercises the release WebView/Java bridge and persistence across process restarts. */
public final class ReadingProgressInstrumentation extends Instrumentation {
    private String phase;

    @Override public void onCreate(Bundle arguments) {
        super.onCreate(arguments);
        phase = arguments.getString("phase", "write");
        start();
    }

    @Override public void onStart() {
        Bundle result = new Bundle();
        try {
            File file = new File(getTargetContext().getFilesDir(), "reading-progress-test.md");
            String documentId = Uri.fromFile(file).toString();
            if ("write".equals(phase)) {
                StringBuilder markdown = new StringBuilder("# Reading progress test\n\n");
                for (int i = 0; i < 200; i++) markdown.append("## Paragraph ").append(i).append("\n\nReading position fixture.\n\n");
                Files.write(file.toPath(), markdown.toString().getBytes(StandardCharsets.UTF_8));
                getTargetContext().getSharedPreferences("reading-progress", 0).edit().remove(documentId).commit();
            }
            Intent intent = new Intent(Intent.ACTION_VIEW, Uri.fromFile(file));
            intent.setClass(getTargetContext(), MainActivity.class);
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_GRANT_READ_URI_PERMISSION);
            Activity activity = startActivitySync(intent);
            WebView web = (WebView) ((android.view.ViewGroup) activity.findViewById(android.R.id.content)).getChildAt(0);
            awaitScript(web, "document.title === 'reading-progress-test.md - MD Preview'");
            String ratio = "(window.scrollY / Math.max(1, document.documentElement.scrollHeight - innerHeight))";
            if ("write".equals(phase)) {
                awaitScript(web, ratio + " < 0.01");
                evaluate(web, "window.dispatchEvent(new Event('wheel')); window.scrollTo(0, (document.documentElement.scrollHeight-innerHeight)*0.6)");
                awaitScript(web, "Math.abs(" + ratio + " - 0.6) < 0.02");
                long deadline = System.currentTimeMillis() + 5000;
                while (Math.abs(getTargetContext().getSharedPreferences("reading-progress", 0).getFloat(documentId, 0f) - .6f) > .02f) {
                    if (System.currentTimeMillis() > deadline) throw new AssertionError("WebView bridge did not persist progress");
                    Thread.sleep(50);
                }
                // Commit the already-written preference so a force-stop in the next command
                // tests disk persistence, independent of the preferences background queue.
                getTargetContext().getSharedPreferences("reading-progress", 0).edit().commit();
            } else {
                awaitScript(web, "Math.abs(" + ratio + " - 0.6) < 0.02");
                // Reopen the persisted Recent Files copy, not the original URI.
                org.json.JSONArray recent = new org.json.JSONArray(getTargetContext().getSharedPreferences("recent", 0).getString("files", "[]"));
                String recentId = recent.getJSONObject(0).getString("id");
                evaluate(web, "window.MDPreview.render({documentId:'other-test-document', name:'Other.md', markdown:'# Other document'})");
                awaitScript(web, ratio + " < 0.01");
                evaluate(web, "window.MDPreviewAndroid.openRecent(" + org.json.JSONObject.quote(recentId) + ")");
                awaitScript(web, "Math.abs(" + ratio + " - 0.6) < 0.02");
                if (!"true".equals(evaluate(web, "document.body.scrollWidth <= innerWidth"))) {
                    throw new AssertionError("Mobile preview overflows viewport");
                }
            }
            runOnMainSync(activity::finish);
            result.putString("stream", "Reading progress " + phase + ": PASS\n");
            finish(Activity.RESULT_OK, result);
        } catch (Throwable error) {
            result.putString("stream", "Reading progress " + phase + ": FAIL: " + error + "\n");
            finish(Activity.RESULT_CANCELED, result);
        }
    }

    private String evaluate(WebView web, String script) throws Exception {
        AtomicReference<String> value = new AtomicReference<>();
        CountDownLatch done = new CountDownLatch(1);
        runOnMainSync(() -> web.evaluateJavascript(script, result -> { value.set(result); done.countDown(); }));
        if (!done.await(5, TimeUnit.SECONDS)) throw new AssertionError("WebView did not respond");
        return value.get();
    }

    private void awaitScript(WebView web, String script) throws Exception {
        long deadline = System.currentTimeMillis() + 10000;
        while (!"true".equals(evaluate(web, script))) {
            if (System.currentTimeMillis() > deadline) throw new AssertionError("Timed out: " + script);
            Thread.sleep(50);
        }
    }
}
