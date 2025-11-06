# summary of potential livereload tools:
# - entr leaves tty available for my tui, but it fails to restart/send SIGTERM
# - watchexec hogs tty so i cannot run a tui, although it restarts and exits well
# - gaze leaves tty available for my tui, and restarts my tui/sends SIGTERM, but error messages are borked (see https://github.com/watchexec/watchexec/issues/194), and i cannot kill the gaze process gracefully
#
# also useful to run in other windows during development:
# - tail -f /tmp/vic_log
# - bacon, for better error/warning messages
#
.PHONY: dev
dev:
	gaze $$(git ls-files) -r -c 'cargo run ./test/bbb_480p_24fps.avi --dry-run --log=/tmp/vic_log'

.PHONY: install
install:
	@echo TODO: install dependencies
	@echo for now, just install test assets
	mkdir -p test
	@# download links taken from: http://bbb3d.renderfarming.net/download.html
	@# be sure to follow redirects, since these links may change over the years
	curl -o test/bbb_480p_24fps.avi -L http://download.blender.org/peach/bigbuckbunny_movies/big_buck_bunny_480p_surround-fix.avi # lite, 220MB
	# curl -o test/bbb_1080p_60fps.mp4 -L http://distribution.bbb3d.renderfarming.net/video/mp4/bbb_sunflower_1080p_60fps_normal.mp4 # heavy, 350MB
	
# perhaps something like:
# ffmpeg create frames containing text label for each frame (1,2,etc)
# ffmpeg create long, colorful .mp4
# ffmpeg create short .mp4 with audio
# ffmpeg create equivalent .opus
#
# not quite as good as testing videos from the wild,
# but that's what gh issues are for i guess
.PHONY: generate-tests
generate-tests:
	@echo todo

# simpler, but worse formatting: @rg TODO
.PHONY: roadmap
roadmap:
	@git ls-files | grep -v Makefile | xargs grep -h TODO | perl -pe 's/^[ \/]*//'
