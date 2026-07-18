-- XIOM Neovim integration: LSP (native vim.lsp) + DAP (nvim-dap).
-- Requires: xiom-lsp and xiom-dbg on PATH (XIOM release bin/).

-- 1. Filetype detection for .xi files
vim.filetype.add({ extension = { xi = 'xiom' } })

-- 2. LSP: start xiom-lsp for xiom buffers
vim.api.nvim_create_autocmd('FileType', {
  pattern = 'xiom',
  callback = function(args)
    vim.lsp.start({
      name = 'xiom-lsp',
      cmd = { 'xiom-lsp' },
      root_dir = vim.fs.root(args.buf, { 'package.xi', '.git' }) or vim.fn.getcwd(),
    })
  end,
})

-- 3. Comment string for gc / commentary
vim.api.nvim_create_autocmd('FileType', {
  pattern = 'xiom',
  callback = function()
    vim.bo.commentstring = '// %s'
  end,
})

-- 4. DAP (optional — requires mfussenegger/nvim-dap)
local ok, dap = pcall(require, 'dap')
if ok then
  dap.adapters.xiom = {
    type = 'executable',
    command = 'xiom-dbg',
  }
  dap.configurations.xiom = {
    {
      type = 'xiom',
      request = 'launch',
      name = 'Debug XIOM program',
      program = function()
        local default = vim.fn.getcwd() .. (vim.fn.has('win32') == 1 and '\\a.exe' or '/a.out')
        return vim.fn.input('Executable: ', default, 'file')
      end,
      cwd = '${workspaceFolder}',
      stopOnEntry = true,
      contractTraps = true,
    },
  }
end
