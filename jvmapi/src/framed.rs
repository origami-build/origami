// needed cause for some reason Framed needs _both_ a reader and a writer at
// once

use std::io;
use std::io::{IoSlice, IoSliceMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use async_codec::{Decode, Encode, Framed, ReadFrameError, WriteFrameError};
use futures::{AsyncRead, AsyncWrite, Sink, Stream};
use pin_project_lite::pin_project;
use std::fmt::Debug;

pin_project! {
    pub struct FramedRead<T, C> {
        #[pin]
        inner: Framed<FakeWriter<T>, C>,
    }
}

impl<T, C> FramedRead<T, C>
    where
        C: Encode + Decode,
{
    pub fn new(reader: T, codec: C) -> Self {
        FramedRead {
            inner: Framed::new(FakeWriter::new(reader), codec),
        }
    }
}

impl<T, C> Stream for FramedRead<T, C>
    where
        T: AsyncRead,
        C: Decode,
{
    type Item = Result<C::Item, ReadFrameError<C::Error>>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }
}

pin_project! {
    pub struct FramedWrite<T, C> {
        #[pin]
        inner: Framed<FakeReader<T>, C>,
    }
}

impl<T, C> FramedWrite<T, C>
    where
        C: Encode + Decode,
{
    pub fn new(writer: T, codec: C) -> Self {
        FramedWrite {
            inner: Framed::new(FakeReader::new(writer), codec),
        }
    }
}

impl<T, C> Sink<C::Item> for FramedWrite<T, C>
    where
        T: AsyncWrite,
        C: Encode,
        C::Error: Debug,
{
    type Error = WriteFrameError<C::Error>;

    #[inline]
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    #[inline]
    fn start_send(self: Pin<&mut Self>, item: C::Item) -> Result<(), Self::Error> {
        self.project().inner.start_send(item)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

pin_project! {
    struct FakeWriter<T> {
        #[pin]
        reader: T
    }
}

impl<T> FakeWriter<T> {
    fn new(reader: T) -> Self {
        FakeWriter { reader }
    }
}

impl<T> AsyncRead for FakeWriter<T>
    where
        T: AsyncRead,
{
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.project().reader.poll_read(cx, buf)
    }

    #[inline]
    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        self.project().reader.poll_read_vectored(cx, bufs)
    }
}

impl<T> AsyncWrite for FakeWriter<T> {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        unreachable!()
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        unreachable!()
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        unreachable!()
    }
}

pin_project! {
    struct FakeReader<T> {
        #[pin]
        pub writer: T
    }
}

impl<T> FakeReader<T> {
    fn new(writer: T) -> Self {
        FakeReader { writer }
    }
}

impl<T> AsyncRead for FakeReader<T> {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        unreachable!()
    }
}

impl<T> AsyncWrite for FakeReader<T>
    where
        T: AsyncWrite,
{
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().writer.poll_write(cx, buf)
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        self.project().writer.poll_write_vectored(cx, bufs)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().writer.poll_flush(cx)
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().writer.poll_close(cx)
    }
}
