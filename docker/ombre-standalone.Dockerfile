# Node for Ombre Alphanet.
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM phusion/baseimage:0.11
LABEL maintainer "contact@shadows.link"
LABEL description="this is the standalone node running Ombre"
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /ombre shadows && \
	mkdir -p /ombre/.local/share/ombre && \
	chown -R shadows:shadows /ombre && \
	ln -s /ombre/.local/share/ombre /data && \
	rm -rf /usr/bin /usr/sbin


USER shadows

COPY --chown=shadows build/standalone /ombre
RUN chmod uog+x /ombre/ombre-standalone

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944 9615

CMD ["/ombre/ombre-standalone", \
	"--dev" \
	"--tmp" \
	"--charlie" \
	"--port","30333", \
	"--rpc-port","9933", \
	"--ws-port","9944", \
	]
