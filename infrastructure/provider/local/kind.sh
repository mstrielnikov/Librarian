#!/bin/bash

set -ex

export KUBECONFIG="${HOME}/.kube/kind/lexicstack"

kind create cluster --name lexicstack --kubeconfig ${KUBECONFIG}

exit 0;