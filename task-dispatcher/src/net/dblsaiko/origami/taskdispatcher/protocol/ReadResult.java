package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public record ReadResult(int tag, int stream, Result<byte[], IoError> result) {
    private static final BinDeserializeFor<Result<byte[], IoError>> RESULT_DES = Result.deserialize(BinDeserializeFor.BYTES, IoError::deserialize);

    public static ReadResult deserialize(BinDeserializer deserializer) throws IOException {
        var tag = deserializer.readU32();
        var stream = deserializer.readU32();
        var result = RESULT_DES.deserialize(deserializer);
        return new ReadResult(tag, stream, result);
    }
}
