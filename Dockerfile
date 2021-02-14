# Inspired by Polkadot Dockerfile

FROM phusion/baseimage:0.11 as builder
LABEL maintainer "contact@shadows.link"
LABEL description="This is the build stage for Shadows. Here we create the binary."

ARG PROFILE=release
WORKDIR /shadows

COPY . /shadows

# Download Shadows repo
RUN apt-get update && \
	apt-get upgrade -y && \
	apt-get install -y cmake pkg-config libssl-dev git clang

# Download rust dependencies and build the rust binary
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH=$PATH:$HOME/.cargo/bin && \
	scripts/init.sh && \
	cargo build --$PROFILE

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer "contact@shadows.link"
LABEL description="This is the 2nd stage: a very small image where we copy the Shadows binary."
ARG PROFILE=release
COPY --from=builder /shadows/target/$PROFILE/node-shadows /usr/local/bin

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /shadows shadows && \
	mkdir -p /shadows/.local/share/shadows && \
	chown -R shadows:shadows /shadows/.local && \
	ln -s /shadows/.local/share/shadows /data && \
	rm -rf /usr/bin /usr/sbin

USER shadows

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944 9615

VOLUME ["/data"]

CMD ["/usr/local/bin/node-shadows"]
