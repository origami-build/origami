package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public interface BinDeserializer {
    int readU8() throws IOException;

    int readU32() throws IOException;

    long readU64() throws IOException;

    String readString() throws IOException;

    int readUsize() throws IOException;
}
