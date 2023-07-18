.PHONY: develop
develop:
	maturin develop --locked -r -E test

.PHONY: test
test: develop
	python -m pytest
