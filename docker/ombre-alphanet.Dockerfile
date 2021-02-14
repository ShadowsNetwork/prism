# Node for Ombre Alphanet.
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM phusion/baseimage:0.11
LABEL maintainer "contact@shadows.link"
LABEL description="this is the parachain node running Ombre Alphanet"
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /ombre-alphanet shadows && \
	mkdir -p /ombre-alphanet/.local/share/ombre-alphanet && \
	chown -R shadows:shadows /ombre-alphanet && \
	ln -s /ombre-alphanet/.local/share/ombre-alphanet /data && \
	rm -rf /usr/bin /usr/sbin


USER shadows

COPY --chown=shadows build/alphanet /ombre-alphanet
RUN chmod uog+x /ombre-alphanet/ombre-alphanet

# 30333 for parachain p2p 
# 30334 for relaychain p2p 
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 30334 9933 9944 9615 

VOLUME ["/data"]

CMD ["/ombre-alphanet/ombre-alphanet", \
	"--chain", "alphanet"\
	]
