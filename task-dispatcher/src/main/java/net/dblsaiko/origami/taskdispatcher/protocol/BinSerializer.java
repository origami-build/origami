package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public interface BinSerializer {
    void writeU8(int u8) throws IOException;

    void writeU32(int u32) throws IOException;

    void writeString(String s) throws IOException;

    void writeUsize(int num) throws IOException;

    void writeBool(boolean bool) throws IOException;
}
