package net.dblsaiko.origami.ojavac;

import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.List;

import javax.tools.JavaCompiler;
import javax.tools.StandardJavaFileManager;
import javax.tools.ToolProvider;

public class Main {
    public static void main(String[] args) {
        JavaCompiler systemJavaCompiler = ToolProvider.getSystemJavaCompiler();
        StandardJavaFileManager standardFileManager = systemJavaCompiler.getStandardFileManager(null, null, StandardCharsets.UTF_8);

        Boolean result = systemJavaCompiler.getTask(
            null,
            standardFileManager,
            null,
            List.of("--source-path", ".", "-implicit:none"),
            null,
            List.of(new JavaFileObjectImpl(Path.of("A.java")))
        ).call();

        System.out.println(result);
    }
}
