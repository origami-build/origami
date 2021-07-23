package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;
import java.time.Duration;
import java.util.Optional;

public record Wait(int tag, int task, Optional<Duration> timeout) {
    private static final BinDeserializeFor<Optional<Duration>> TIMEOUT_DES = BinDeserializeFor.option(BinDeserializeFor.DURATION);

    public static Wait deserialize(BinDeserializer deserializer) throws IOException {
        var tag = deserializer.readU32();
        var task = deserializer.readU32();
        var timeout = TIMEOUT_DES.deserialize(deserializer);
        return new Wait(tag, task, timeout);
    }
}
