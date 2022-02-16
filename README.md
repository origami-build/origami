# origami

Origami is intended as a collection of tools to build Java projects, especially
Minecraft mods. This is still heavily in development and can't currently be used
except to build very simple projects.

## Components

- [ojavac](java-compilers/ojavac#readme), the javac wrapper
- ojvmd, the host for running multiple Java programs in the same JVM
- omake, the build tool
- oresolve, the dependency resolver

(+ possibly more in the future)

## Requirements

- make and coreutils
- a recent version of nightly Rust/cargo
- Java JDK 16 or higher

## Building and Installation

Currently, building is only tested on Linux. It might work on Mac and will
definitely not work on Windows without any changes to the Makefiles (coming
soon™).

To build a system installation and install it to /usr/local, use

    make && make install

To build an installation for your local user (assuming $HOME/.local/bin) is in
your PATH, use (shortened to one command for brevity, specify prefix= for both
the compilation and installation command)

    make prefix=$HOME/.local all install

You can customize the directory to be installed and the installation prefix, for
example, to package for a distribution you might use

    make prefix=/usr DESTDIR=dist/ all install

to compile for installation to /usr and install all files in a subdirectory of
dist instead of directly to /.

It is also possible to compile a "portable" installation:

    make prefix=. bindir=. DESTDIR=dist/ all install

The dist/ directory should now be able to be placed anywhere and origami should
find its files.