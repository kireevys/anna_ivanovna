check-tag:
	scripts/check-tag-version.sh

new-release: # 	scripts/create-release.sh version=1.0.0
	scripts/create-release.sh $(version)