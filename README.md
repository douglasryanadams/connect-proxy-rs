# connect-proxy-rs

This is an HTTP Proxy with a very specific use case, it's not a swiss army knife. It allows you to forward packets through a remote hop for any TCP traffic that happens after an HTTP CONNECT message. This narrow use case covers the majority of modern web sites that support HTTPS over either HTTP/1.1 or HTTP/2 and is supported by most HTTP clients.

# What it does

This proxy will read only the first line of the CONNECT message to make sure that it is a CONNECT message and to discover the domain of the target site. Once it's read those two parts of the initial message it will stop parsing the HTTP traffic, establish a TCP connection with the target site and let data flow freely through the proxy.

# What it does not do

- This proxy does not support HTTP requests for other method types.
- This proxy does not support any other proxy-like protocols (SOCKS for example)

# Roadmap

[x] Hello World
[ ] Command line arguments to set listener port and IP
[ ] Bind outgoing traffic to the same listener port and IP
[ ] DNS Resolution from Proxy (Not Tested)
[ ] DNS Resolution from Proxy to specific DNS Server
[ ] DNS Resolution from the same listener port and IP
[ ] More robust complete of other narrow things CONNECT can support (like Keepalive)

