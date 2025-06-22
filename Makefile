check-tag:
	scripts/check-tag-version.sh

release: # 	make release v=1.0.0
	scripts/create-release.sh $(v)
