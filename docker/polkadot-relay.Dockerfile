# Inspired by Polkadot Dockerfile

FROM phusion/baseimage:0.11 as builder
LABEL maintainer "contact@shadows.link"
LABEL description="This is the build stage for Polkadot. Here we create the binary."

ARG PROFILE=release
ARG POLKADOT_COMMIT=master
RUN echo "Using polkadot ${POLKADOT_COMMIT}"
WORKDIR /

# Install OS dependencies
RUN apt-get update && \
	apt-get upgrade -y && \
	apt-get install -y cmake pkg-config libssl-dev git clang

# Grab the Polkadot Code
# TODO how to grab the correct commit from the lock file?
RUN git clone https://github.com/paritytech/polkadot
WORKDIR /polkadot
RUN git checkout ${POLKADOT_COMMIT}

# Forces to use the compiled wasm engine for parachain validation
RUN sed -i '/sc_executor::WasmExecutionMethod::Interpreted/c\\t\tsc_executor::WasmExecutionMethod::Compiled,' parachain/src/wasm_executor/mod.rs

# Download rust dependencies and build the rust binary
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH=$PATH:$HOME/.cargo/bin && \
	scripts/init.sh && \
	cargo build --$PROFILE --features=real-overseer

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer "contact@shadows.link"
LABEL description="Polkadot for Shadows Alphanet Relay Chain"
ARG PROFILE=release
COPY --from=builder /polkadot/target/$PROFILE/polkadot /usr/local/bin

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

CMD ["/usr/local/bin/polkadot"]
