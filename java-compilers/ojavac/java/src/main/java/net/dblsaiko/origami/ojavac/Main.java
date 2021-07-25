package net.dblsaiko.origami.ojavac;

import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.List;

import javax.tools.JavaCompiler;
import javax.tools.JavaCompiler.CompilationTask;
import javax.tools.ToolProvider;

public class Main {
    public static void main(String[] args) {
        JavaCompiler systemJavaCompiler = ToolProvider.getSystemJavaCompiler();
        TrackingJavaFileManager fm = new TrackingJavaFileManager(
            systemJavaCompiler.getStandardFileManager(null, null, StandardCharsets.UTF_8)
        );

        CompilationTask task = systemJavaCompiler.getTask(
            null,
            fm,
            null,
            List.of("--source-path", ".", "-implicit:none", "-classpath", "/home/saiko/src/origami/ecj-3.25.0.jar"),
            null,
            List.of(new JavaFileObjectImpl(Path.of("A.java")))
        );
        Boolean result = task.call();

        if (result) {
            for (Path inputFile : fm.getInputFiles()) {
                System.out.println("in: " + inputFile);
            }

            for (Path outputFile : fm.getOutputFiles()) {
                System.out.println("out: " + outputFile);
            }
        }

        System.out.println(result);
    }
}
