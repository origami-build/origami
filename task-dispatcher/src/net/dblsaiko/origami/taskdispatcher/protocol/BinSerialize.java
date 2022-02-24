package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public interface BinSerialize {
    void serialize(BinSerializer serializer) throws IOException;
}
