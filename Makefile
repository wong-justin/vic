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

# TODO: automate obs recording as well
.PHONY: demo
demo:
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
	'T("<CR>", 600) '\
	'Show() '\
	'T("?", 1500) '\
	'T("8", 2500) '\
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
	'T("q", 1500) '\
	'Hide() '\
	'T("<ESC>", 1500) '\
	'T(":q<CR>", 1000) '
