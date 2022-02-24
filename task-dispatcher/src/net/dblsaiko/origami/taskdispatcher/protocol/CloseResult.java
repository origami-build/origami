package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public record CloseResult(int tag, Result<Unit, IoError> result) {
    private static final BinDeserializeFor<Result<Unit, IoError>> RESULT_DES = Result.deserialize(Unit::deserialize, IoError::deserialize);

    public static CloseResult deserialize(BinDeserializer deserializer) throws IOException {
        var tag = deserializer.readU32();
        var result = RESULT_DES.deserialize(deserializer);
        return new CloseResult(tag, result);
    }
}
