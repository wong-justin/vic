.PHONY: dev
dev:
	@# summary of potential livereload tools:
	@# - entr leaves tty available for my tui, but it fails to restart/send SIGTERM
	@# - watchexec hogs tty so i cannot run a tui, although it restarts and exits well
	@# - gaze leaves tty available for my tui, and restarts my tui/sends SIGTERM, but error messages are borked (see https://github.com/watchexec/watchexec/issues/194), and i cannot kill the gaze process gracefully
	@#
	@# also useful to run in other shells during development:
	@# - tail -f /tmp/vic_log
	@# - bacon, for better error/warning messages
	gaze $$(git ls-files) -r -c 'cargo run ./test/bbb_480p_24fps.avi --dry-run --log=/tmp/vic_log'

.PHONY: install-tests
install-tests:
	@# TODO: install dependencies. for now, just install some test assets
	mkdir -p test
	@# download links taken from: http://bbb3d.renderfarming.net/download.html
	@# be sure to follow redirects, since these links may change over the years
	curl -o test/bbb_480p_24fps.avi -L http://download.blender.org/peach/bigbuckbunny_movies/big_buck_bunny_480p_surround-fix.avi # liter, 220MB
	# curl -o test/bbb_1080p_60fps.mp4 -L http://distribution.bbb3d.renderfarming.net/video/mp4/bbb_sunflower_1080p_60fps_normal.mp4 # heavy, 350MB

.PHONY: generate-tests
generate-tests:
	@# TODO: perhaps generate some test videos like:
	@# ffmpeg create frames containing text label for each frame (1,2,etc)
	@# ffmpeg create long, colorful .mp4
	@# ffmpeg create short .mp4 with audio
	@# ffmpeg create equivalent .opus
	@#
	@# not quite as good as testing videos from the wild,
	@# but that's what gh issues are for i guess

.PHONY: roadmap
roadmap:
	@# simpler, but worse formatting: @rg TODO
	@git ls-files | grep -v Makefile | xargs grep -h TODO | perl -pe 's/^[ \/]*//'

.PHONY: build-static
build-static:
	podman --cgroup-manager=cgroupfs build .
	@# podman cp $(podman create vicbuild):/app/target/release/vic ./vic_static_x86_64_linux

.PHONY: demo
demo:
	@# TODO: fully automate obs recording (scenes, startup, splicing)
	@#
	uvx obs-cli record start
	@# https://github.com/wong-justin/showkeys-noplug
	@nvim -c 'lua '\
	'recent_keypresses={} '\
	'max_recent_keypresses=3 '\
	'msg = " " '\
	'width = vim.fn.strdisplaywidth(msg) '\
	'buf = vim.api.nvim_create_buf(false, true) '\
	'window_config = {relative="editor",width=width,height=1,col=0,row=1,style="minimal",border="solid"} '\
	'window_id = nil '\
	'hidden = false '\
	'ns = vim.api.nvim_create_namespace("showkeysforknamespace") '\
	'vim.api.nvim_set_hl(0, "PastHighlight", { default = true, link = "Visual" }) '\
	'vim.api.nvim_set_hl(0, "CurrentHighlight", { default = true, link = "pmenusel" }) '\
	'normal = vim.api.nvim_get_hl(0, { name = "Normal" }) '\
	'vim.api.nvim_set_hl(0, "Normal", { bg = "NONE", fg=normal.fg }) '\
	'vim.api.nvim_set_hl(0, "Error", {}) '\
	'vim.api.nvim_set_hl(0, "DiagnosticUnderlineError", {}) '\
	'vim.api.nvim_set_hl(0, "DiagnosticUnderlineWarn", {}) '\
	'vim.api.nvim_set_hl(0, "DiagnosticUnderlineInfo", {}) '\
	'vim.api.nvim_set_hl(0, "DiagnosticUnderlineHint", {}) '\
	'vim.api.nvim_set_hl(0, "SpellCap", {}) '\
	'vim.api.nvim_set_hl(0, "SpellLocal", {}) '\
	'vim.on_key(function(_, char) '\
	'  if hidden then '\
	'    if window_id ~= nil then vim.api.nvim_win_close(window_id, true); end '\
	'    window_id = nil '\
	'    recent_keypresses = {} '\
	'    return '\
	'  end '\
	'  if window_id == nil then window_id = vim.api.nvim_open_win(buf, false, window_config); end '\
	'  vim.wo[window_id].winhl = "FloatBorder:Normal,Normal:Normal" '\
	'  key = vim.fn.keytrans(char) '\
	'  special_keys = {["<BS>"] = "󰁮 ",["<CR>"] = "󰘌",["<Space>"] = "󱁐",["<Up>"] = "󰁝",["<Down>"] = "󰁅",["<Left>"] = "󰁍",["<Right>"] = "󰁔",["<PageUp>"] = "Page 󰁝",["<PageDown>"] = "Page 󰁅",["<M>"] = "Alt",["<C>"] = "Ctrl"} '\
	'  msg = special_keys[key] or key '\
	'  table.insert(recent_keypresses, msg) '\
	'  if #recent_keypresses > max_recent_keypresses then table.remove(recent_keypresses, 1) end '\
	'  display_str = "  " .. table.concat(recent_keypresses, "   ") .. "  " '\
	'  width = vim.fn.strdisplaywidth(display_str) '\
	'  window_config.width = width '\
	'  window_config.col = math.floor((vim.o.columns - width) / 2) '\
	'  vim.api.nvim_win_set_config(window_id, window_config) '\
	'  vim.api.nvim_buf_set_lines(buf, 0, -1, false, {display_str}) '\
	'  last_pos=1 '\
	'  for i = 1, #recent_keypresses do '\
	'    this_width = vim.fn.strlen(recent_keypresses[i]) '\
	'    hl = "PastHighlight"; if i == #recent_keypresses then hl = "CurrentHighlight" end '\
	'    vim.hl.range(buf, ns, hl, {0,last_pos}, {0,last_pos+this_width+2}) '\
	'    last_pos = last_pos + 1 + this_width + 2 '\
	'  end '\
	'end) '\
	'time = 0 '\
	'T = function(inputs, delay_ms) vim.fn.timer_start(time + delay_ms, function() vim.api.nvim_input(inputs) end) time = time + delay_ms; end '\
	'Hide = function() vim.fn.timer_start(time, function() hidden = true; end) end '\
	'Show = function() vim.fn.timer_start(time, function() hidden = false; end) end '\
	'vim.cmd("terminal") '\
	''\
	'Hide() '\
	'T("a", 0) '\
	'T("vic", 200) '\
	'T(" t", 200) '\
	'T("est/", 200) '\
	'T("b", 200) '\
	'T("bb_480p_24fps.avi", 200) '\
	'T(" -w", 200) '\
	'T(" 999", 200) '\
	'T(" --dry-run", 200) '\
	'T("<CR>", 400) '\
	'Show() '\
	'T("?", 1000) '\
	'T("l", 2000) '\
	'T("l", 1000) '\
	'T("l", 800) '\
	'T("m", 800) '\
	'T("8", 2000) '\
	'T("<Left>", 1500) '\
	'T("<Left>", 500) '\
	'T("<Left>", 500) '\
	'T("<Left> ", 500) '\
	'T(".", 200) '\
	'T(".", 200) '\
	'T(".", 200) '\
	'T(".", 200) '\
	'T(".", 200) '\
	'T(".", 200) '\
	'T(".", 200) '\
	'T(".", 800) '\
	'T("m", 800) '\
	'T("J", 1200) '\
	'T("L", 1000) '\
	'T("J", 1000) '\
	'T("M", 1000) '\
	'T(" ", 800) '\
	'T("q", 2000) '\
	'Hide() '\
	'T("<C-l>", 1500) '\
	'T("#uvx obs-cli record stop<CR>", 1500) '\
	'T("<ESC>", 1000) '\
	'' # 'T(":q<CR>", 1000) '
