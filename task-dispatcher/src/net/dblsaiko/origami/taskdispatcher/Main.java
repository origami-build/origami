package net.dblsaiko.origami.taskdispatcher;

import java.io.IOException;
import java.io.InputStream;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;

public class Main {
    public static void main(String[] args) throws IOException {
        // set up System.out/System.in so that output from tasks is packed
        // appropriately
        PrintStream stdout = System.out;
        PrintStream stderr = System.err;
        InputStream stdin = System.in;

        var pw = new ProtocolWriter(stdout);
        var pr = new ProtocolReader(stdin);
        var ctl = new CommController(pw, pr);

        var stream = new MuxOutputStream(WrappedOutputStream.create(-1, ctl));
        var estream = new MuxOutputStream(stderr);
        var istream = new MuxInputStream(null);
        System.setOut(new PrintStream(stream, true, StandardCharsets.UTF_8));
        System.setErr(new PrintStream(estream, true, StandardCharsets.UTF_8));
        System.setIn(istream);

        ctl.readerLoop(new Dispatcher(stream, estream, istream, ctl));
    }
}
