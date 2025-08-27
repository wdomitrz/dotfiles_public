--------------------------------------------------------------------------------
-- Copyright (c) 2025 Witalis Domitrz <witekdomitrz@gmail.com>
-- AGPL License
--------------------------------------------------------------------------------
-- Based on https://github.com/rktjmp/fwatch.nvim (MIT license Oliver Marriott)

local theme_file_path = vim.fn.expand("$HOME/.config/nvim/theme.txt")

local function read_file(path)
    local file = io.open(path, "r")
    if not file then
        return "dark"
    end
    local content = file:read("*line")
    file:close()
    return content
end

local function set_theme()
    local theme = read_file(theme_file_path)
    if theme == "light" then
        vim.api.nvim_set_option_value("background", "light", {})
    else
        vim.api.nvim_set_option_value("background", "dark", {})
    end
end

local function watch_theme_change()
    local function event_cb(err)
        if err then
        else
            vim.schedule(set_theme)
        end
    end

    local handle = vim.uv.new_fs_event()
    if handle then
        vim.uv.fs_event_start(
            handle,
            theme_file_path,
            {
                watch_entry = false,
                stat = false,
                recursive = false,
            },
            event_cb
        )
    end

    return handle
end

set_theme()
watch_theme_change()
