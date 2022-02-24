package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public enum Stdio {
    NULL,
    PIPED,
    INHERIT;

    public static Stdio deserialize(BinDeserializer deserializer) throws IOException {
        int variant = deserializer.readUsize();
        return Stdio.values()[variant];
    }
}
