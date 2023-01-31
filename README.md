# druid-pane-demo

A simple demo that demonstrates the use of druid for a pane dock.

A pane dock is a contained area where "pane" windows are organized on the bottom right.
Panes can be moved around, and closed.

A future goal is to allow them to also be resized and minimized.

### Building and running

Running this, once dependencies are resolved, is as simple as running `cargo run`

The most important dependencies are `rust` and `cargo`, but on Linux you may need to also install
some other packages to resolve linking errors.

On Fedora, the packages you need to install are one of:
```
gtk2-devel
webkit2gtk3-devel
```

On ubuntu and debian:
```
libgtk-3-dev
```

On FreeBSD:
```
pkg_add gtk+3
```

On Mac OS, you probably need to install xcode.