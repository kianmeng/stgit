prefix	?= $(HOME)
DESTDIR	?= /
PYTHON	?= python

export DESTDIR PYTHON

TEST_PATCHES ?= ..

all:
	$(PYTHON) setup.py build

install:
	$(PYTHON) setup.py install --prefix=$(prefix) --root=$(DESTDIR) --force

doc:
	$(MAKE) -C Documentation all

install-doc:
	$(MAKE) -C Documentation install

install-html:
	$(MAKE) -C Documentation install-html

lint:
	$(PYTHON) -m flake8 . stg stg-build stg-dbg stg-prof

test:
	$(PYTHON) setup.py build
	$(MAKE) -C t all

test_patches:
	for patch in $$(stg series --noprefix $(TEST_PATCHES)); do \
		stg goto $$patch && $(MAKE) test || break; \
	done

coverage: coverage-test coverage-report

coverage-test:
	$(PYTHON) -m coverage run setup.py build
	COVERAGE_PROCESS_START=$(PWD)/.coveragerc $(MAKE) -C t all
	$(PYTHON) -m coverage combine $$(find . -name '.coverage.*')

coverage-report:
	$(PYTHON) -m coverage html --title="stgit coverage"
	$(PYTHON) -m coverage report
	@echo "HTML coverage report: file://$(PWD)/htmlcov/index.html"

clean:
	for dir in Documentation t; do \
		$(MAKE) -C $$dir clean; \
	done
	rm -rf build
	rm -rf dist
	rm  -f stgit/*.pyc
	rm -rf stgit/__pycache__
	rm  -f stgit/builtin_version.py
	rm  -f stgit/commands/*.pyc
	rm -rf stgit/commands/__pycache__
	rm  -f stgit/commands/cmdlist.py
	rm  -f stgit/lib/*.pyc
	rm -rf stgit/lib/__pycache__
	rm  -f TAGS tags
	rm  -f MANIFEST
	rm  -f stgit-completion.bash

tags:
	ctags -R stgit/*

TAGS:
	ctags -e -R stgit/*

.PHONY: all install doc install-doc install-html test test_patches \
	lint coverage coverage-test coverage-report clean tags TAGS
