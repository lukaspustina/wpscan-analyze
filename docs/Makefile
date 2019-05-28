
all: man1/wpscan-analyze.1

man1/wpscan-analyze.1: wpscan-analyze.1.md man1 pandoc
	@pandoc -s -t man $< -o $@
	@sed -i.bk 's/\.TH.*/.TH "wpscan-analyze" "1"/g' man1/wpscan-analyze.1
	@rm -f man1/wpscan-analyze.1.bk

man1:
	@mkdir $@

pandoc:
	@pandoc --version > /dev/null

view: man1/wpscan-analyze.1
	@man -M . 1 wpscan-analyze

clean:
	@-rm -R man1

.PHONY: pandoc view clean

