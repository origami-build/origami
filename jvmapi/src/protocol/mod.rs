use std::io;
use std::io::{Cursor, ErrorKind, Read, Write};

use async_codec::{Decode, DecodeResult, Encode, EncodeResult};

pub mod fncall;
pub mod streams;
pub mod structs;

pub struct ProtocolCodec;

impl Encode for ProtocolCodec {
    type Item = structs::ToJvm<'static>;
    type Error = binserde::Error;

    fn encode(&mut self, item: &Self::Item, buf: &mut [u8]) -> EncodeResult<Self::Error> {
        struct Counter<W> {
            inner: W,
            inner_err: Option<io::Error>,
            bytes_read: usize,
            bytes_written: usize,
        }

        impl<W> Write for Counter<W>
        where
            W: Write,
        {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.bytes_read += buf.len();

                if self.inner_err.is_none() {
                    let result = self.inner.write(buf);

                    match result {
                        Ok(len) => {
                            self.bytes_written += len;
                            Ok(len)
                        }
                        Err(e) => {
                            self.inner_err = Some(e);
                            Ok(buf.len())
                        }
                    }
                } else {
                    Ok(buf.len())
                }
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let mut writer = Counter {
            inner: Cursor::new(buf),
            inner_err: None,
            bytes_read: 0,
            bytes_written: 0,
        };

        match binserde::serialize_into(&mut writer, item) {
            Ok(_) => match writer.inner_err {
                None => EncodeResult::Ok(writer.bytes_written),
                Some(e) => {
                    if writer.bytes_read > writer.inner.get_ref().len() {
                        EncodeResult::Overflow(writer.bytes_read)
                    } else {
                        EncodeResult::Err(binserde::Error::Io(e))
                    }
                }
            },
            Err(binserde::Error::Io(e)) if e.kind() == ErrorKind::WriteZero => {
                EncodeResult::Overflow(writer.bytes_read)
            }
            Err(e) => EncodeResult::Err(e),
        }
    }
}

impl Decode for ProtocolCodec {
    type Item = structs::FromJvm;
    type Error = binserde::Error;

    fn decode(&mut self, buffer: &mut [u8]) -> (usize, DecodeResult<Self::Item, Self::Error>) {
        struct EofTracker<'a> {
            buffer: Cursor<&'a [u8]>,
            hit_eof: bool,
        }

        impl<'a> Read for EofTracker<'a> {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                // println!(
                //     "buf.len() = {}, self.buffer.position() = {}, self.buffer.get_ref.len() = {}",
                //     buf.len(),
                //     self.buffer.position(),
                //     self.buffer.get_ref().len()
                // );
                if buf.len() > 0 && self.buffer.position() == self.buffer.get_ref().len() as u64 {
                    self.hit_eof = true;
                }

                self.buffer.read(buf)
            }
        }

        // for x in &*buffer {
        //     eprint!("{:02X}", x);
        // }
        //
        // eprintln!();

        let mut reader = EofTracker {
            buffer: Cursor::new(buffer),
            hit_eof: false,
        };

        match binserde::deserialize_from(&mut reader) {
            Ok(v) => (reader.buffer.position() as usize, DecodeResult::Ok(v)),
            Err(e) => {
                if reader.hit_eof {
                    (0, DecodeResult::UnexpectedEnd)
                } else {
                    (0, DecodeResult::Err(e))
                }
            }
        }
    }
}
