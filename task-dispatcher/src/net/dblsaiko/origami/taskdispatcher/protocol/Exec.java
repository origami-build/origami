package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;
import java.util.OptionalInt;

public record Exec(int tag, String mainClass, String[] params, OptionalInt stdout, OptionalInt stderr, OptionalInt stdin) {
    private static final BinDeserializeFor<String[]> PARAMS_DES = BinDeserializeFor.array(String[]::new, BinDeserializer::readString);

    public static Exec deserialize(BinDeserializer deserializer) throws IOException {
        var tag = deserializer.readU32();
        var mainClass = deserializer.readString();
        var params = PARAMS_DES.deserialize(deserializer);
        var stdout = BinDeserializeFor.OPTION_U32.deserialize(deserializer);
        var stderr = BinDeserializeFor.OPTION_U32.deserialize(deserializer);
        var stdin = BinDeserializeFor.OPTION_U32.deserialize(deserializer);
        return new Exec(tag, mainClass, params, stdout, stderr, stdin);
    }
}
