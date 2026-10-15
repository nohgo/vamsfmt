-- Merge these entries into your existing conform.nvim setup.
-- Install the vamsfmt executable on PATH first; see README.md.
return {
  'stevearc/conform.nvim',
  opts = {
    formatters = {
      vamsfmt = {
        command = 'vamsfmt',
        args = { '--indent-width', '2', '--line-width', '100' },
        stdin = true,
      },
    },
    formatters_by_ft = { verilogams = { 'vamsfmt' } },
    format_on_save = { timeout_ms = 1000, lsp_format = 'never' },
  },
}
