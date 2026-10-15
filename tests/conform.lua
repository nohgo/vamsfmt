-- Real Conform integration in a clean Neovim session; all files stay in build/.
local root = vim.fn.getcwd()
local dependency = root .. '/build/test-deps/conform.nvim'
assert(vim.fn.isdirectory(dependency) == 1, 'Fetch the documented pinned Conform test dependency first')
vim.opt.runtimepath:prepend(dependency)
local conform = require('conform')
conform.setup({
  formatters = { vamsfmt = {
    command = root .. '/target/release/vamsfmt',
    stdin = true,
  } },
  formatters_by_ft = { verilogams = { 'vamsfmt' } },
  format_on_save = { timeout_ms = 10000, lsp_format = 'never' },
  notify_on_error = false,
})
local buf = vim.api.nvim_create_buf(true, false)
vim.api.nvim_set_current_buf(buf)
vim.api.nvim_buf_set_name(buf, root .. '/build/conform-test.va')
vim.bo[buf].filetype = 'verilogams'
local compact = { 'module m;electrical p;analog begin I(p)<+1;end endmodule' }
local expected = { 'module m;', '  electrical p;', '  analog begin', '    I(p) <+ 1;', '  end', 'endmodule' }
vim.api.nvim_buf_set_lines(buf, 0, -1, false, compact)
local callback_called = false
conform.format({ bufnr = buf, async = false, timeout_ms = 10000 }, function(err, changed)
  assert(not err, err)
  assert(changed)
  callback_called = true
end)
assert(callback_called)
assert(vim.deep_equal(vim.api.nvim_buf_get_lines(buf, 0, -1, false), expected))
-- BufWritePre invokes the actual format_on_save hook and writes formatted text.
vim.api.nvim_buf_set_lines(buf, 0, -1, false, compact)
vim.cmd('silent write!')
assert(vim.deep_equal(vim.fn.readfile(root .. '/build/conform-test.va'), expected))
-- Invalid source must fail without replacing any buffer content.
local broken = { 'module m;', '  analog begin I(p) <+ ; end', 'endmodule' }
vim.api.nvim_buf_set_lines(buf, 0, -1, false, broken)
local rejected = false
conform.format({ bufnr = buf, async = false, quiet = true, timeout_ms = 10000 }, function(err)
  assert(err and err:find('vamsfmt', 1, true))
  rejected = true
end)
assert(rejected)
assert(vim.deep_equal(vim.api.nvim_buf_get_lines(buf, 0, -1, false), broken))
vim.cmd('silent write!')
assert(vim.deep_equal(vim.api.nvim_buf_get_lines(buf, 0, -1, false), broken))
assert(vim.deep_equal(vim.fn.readfile(root .. '/build/conform-test.va'), broken))
print('Conform: formatted buffer and saved file; rejected invalid input with buffer unchanged')
vim.cmd('qa!')
