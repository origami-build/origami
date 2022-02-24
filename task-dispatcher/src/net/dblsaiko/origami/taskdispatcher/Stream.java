package net.dblsaiko.origami.taskdispatcher;

import java.io.IOException;

public interface Stream<T> {
    T next() throws IOException;
}
