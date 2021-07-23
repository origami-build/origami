package net.dblsaiko.origami.taskdispatcher;

import java.io.IOException;

public interface Sink<T> {
    void send(T t) throws IOException;
}
