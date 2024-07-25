NAME = tutorial
FILE = configs/osmo-juno.yaml

HELM_REPO = starship
HELM_CHART = devnet
HELM_VERSION = 0.2.3

###############################################################################
###                              All commands                               ###
###############################################################################

.PHONY: setup
setup: setup-deps setup-helm setup-kind

.PHONY: stop
stop: stop-forward delete

.PHONY: clean
clean: stop clean-kind

###############################################################################
###                          Dependency check                               ###
###############################################################################

.PHONY: setup-deps
setup-deps:
	npm install -g @starship-ci/cli
	starship install

###############################################################################
###                              Helm Charts                                ###
###############################################################################

setup-helm:
	starship setup

install:
	starship start --config $(FILE)

install-tiny:
	$(MAKE) install FILE=$(TINY_FILE)

delete:
	starship stop

###############################################################################
###                                 Port forward                            ###
###############################################################################

.PHONY: port-forward
port-forward:
	bash $(CURDIR)/port-forward.sh --config=$(FILE)

.PHONY: stop-forward
stop-forward:
	-pkill -f "port-forward"

###############################################################################
###                          Local Kind Setup                               ###
###############################################################################
KIND_CLUSTER=starship

.PHONY: setup-kind
setup-kind:
	kind create cluster --name $(KIND_CLUSTER)

.PHONY: watch-pods
watch-pods:
	watch kubectl get pods

.PHONY: clean-kind
clean-kind:
	kind delete cluster --name $(KIND_CLUSTER)
