# Node for Shadows Parachains.

FROM phusion/baseimage:0.11
LABEL maintainer "contact@shadows.link"
LABEL description="This is the build dev for Shadows. Here we create the binary."

ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /shadows-parachain shadows && \
	mkdir -p /shadows-parachain/.local/share/shadows-parachain && \
	chown -R shadows:shadows /shadows-parachain && \
	ln -s /shadows-parachain/.local/share/shadows-parachain /data && \
	rm -rf /usr/bin /usr/sbin

USER shadows

COPY --chown=shadows bin /shadows-parachain
RUN chmod uog+x /shadows-parachain/shadows-node

# 30333 for parachain p2p
# 30334 for relaychain p2p
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 30334 9933 9944 9615

VOLUME ["/data"]

CMD ["/shadows-parachain/shadows-node","--dev"]
