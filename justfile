# pull-framework:
#     git subtree pull --prefix=framework frameworks main

# push-framework {{branch}}:
#     git subtree pull --prefix=framework frameworks {{branch}}

# pull-adapters:
#     git subtree pull --prefix=adapters adapters main

# push-adapters {{branch}}:
#     git subtree pull --prefix=adapters adapters {{branch}}

# pull-apps:
#     git subtree pull --prefix=apps apps main

# push-apps {{branch}}:
#     git subtree pull --prefix=apps apps {{branch}}

pull {{repo}}:
    git subtree pull --prefix={{repo}} {{repo}} main

push {{repo}} {{branch}}:
    git subtree pull --prefix={{repo}} {{repo}} {{branch}}