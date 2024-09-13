all: csv_gp_python/LICENSE csv_gp_python/README.md

csv_gp_python/LICENSE: LICENSE
	cp LICENSE csv_gp_python
csv_gp_python/README.md:
	cp README.md csv_gp_python
