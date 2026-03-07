rendered/tile.html: rendered/tile-topzonet1t2.svg rendered/tile-botzonet1t2.svg rendered/tile-topzonet3t4.svg rendered/tile-botzonet3t4.svg \
	rendered/tile-wor-imux_2.svg rendered/tile-wor-imux_1.svg rendered/tile-xnor1-imux_2.svg rendered/tile-xnor1-imux_1.svg \
	rendered/tile-ff-imux_2.svg rendered/tile-ff-imux_1.svg rendered/tile-xnor2-imux_2.svg rendered/tile-xnor2-imux_1.svg \
	rendered/tile-wor-botbmux.svg rendered/tile-wor-topbmux.svg \
	rendered/tile-xnor1-botbmux.svg rendered/tile-xnor1-topbmux.svg \
	templates/tile.tera
	cargo run

clean:
	rm rendered/*.svg

rendered/tile-topzonet1t2.svg: templates/tile-topzonet1t2.json
	npx bit-field -i $< --vflip --hspace 1920 --lanes 1 > $@

rendered/tile-botzonet1t2.svg: templates/tile-botzonet1t2.json
	npx bit-field -i $< --vflip --hspace 1920 --lanes 1 > $@

rendered/tile-topzonet3t4.svg: templates/tile-topzonet3t4.json
	npx bit-field -i $< --vflip --hspace 1920 --lanes 1 > $@

rendered/tile-botzonet3t4.svg: templates/tile-botzonet3t4.json
	npx bit-field -i $< --vflip --hspace 1920 --lanes 1 > $@

rendered/tile-wor-imux_2.svg: templates/tile-wor-imux_2.json
	npx bit-field -i $< --vflip --hspace 640 --offset 0 --lanes 1 > $@

rendered/tile-xnor1-imux_2.svg: templates/tile-xnor1-imux_2.json
	npx bit-field -i $< --vflip --hspace 640 --offset 12 --lanes 1 > $@

rendered/tile-ff-imux_2.svg: templates/tile-ff-imux_2.json
	npx bit-field -i $< --vflip --hspace 640 --offset 0 --lanes 1 > $@

rendered/tile-xnor2-imux_2.svg: templates/tile-xnor2-imux_2.json
	npx bit-field -i $< --vflip --hspace 640 --offset 12 --lanes 1 > $@

rendered/tile-wor-imux_1.svg: templates/tile-wor-imux_1.json
	npx bit-field -i $< --vflip --hspace 640 --offset 0 --lanes 1 > $@

rendered/tile-xnor1-imux_1.svg: templates/tile-xnor1-imux_1.json
	npx bit-field -i $< --vflip --hspace 640 --offset 12 --lanes 1 > $@

rendered/tile-ff-imux_1.svg: templates/tile-ff-imux_1.json
	npx bit-field -i $< --vflip --hspace 640 --offset 0 --lanes 1 > $@

rendered/tile-xnor2-imux_1.svg: templates/tile-xnor2-imux_1.json
	npx bit-field -i $< --vflip --hspace 640 --offset 12 --lanes 1 > $@

rendered/tile-wor-botbmux.svg: templates/tile-wor-botbmux.json
	npx bit-field -i $< --vflip --hspace 240 --offset 9 --lanes 1 > $@

rendered/tile-wor-topbmux.svg: templates/tile-wor-topbmux.json
	npx bit-field -i $< --vflip --hspace 240 --offset 9 --lanes 1 > $@

rendered/tile-xnor1-botbmux.svg: templates/tile-xnor1-botbmux.json
	npx bit-field -i $< --vflip --hspace 240 --offset 9 --lanes 1 > $@

rendered/tile-xnor1-topbmux.svg: templates/tile-xnor1-topbmux.json
	npx bit-field -i $< --vflip --hspace 240 --offset 9 --lanes 1 > $@

.SUFFIXES:
