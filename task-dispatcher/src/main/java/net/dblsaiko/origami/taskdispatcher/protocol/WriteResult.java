package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public record WriteResult(int tag, Result<Integer, IoError> result) {
    private static final BinSerializeFor<Result<Integer, IoError>> RESULT_SER = Result.serialize(BinSerializeFor.USIZE, IoError::serialize);
    private static final BinDeserializeFor<Result<Integer, IoError>> RESULT_DES = Result.deserialize(BinDeserializeFor.USIZE, IoError::deserialize);

    public static WriteResult deserialize(BinDeserializer deserializer) throws IOException {
        var tag = deserializer.readU32();
        var result = RESULT_DES.deserialize(deserializer);
        return new WriteResult(tag, result);
    }
}
