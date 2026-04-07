#compdef vaulter

_vaulter() {
    local context state state_descr line
    typeset -A opt_args

    _arguments -C \
        '1: :_vaulter_commands' \
        '*:: :->args'

    case $state in
        args)
            case $words[1] in
                switch|sw|delete|rm)
                    _arguments '1:vault:_vaulter_vaults'
                    ;;
                show|print|use|select)
                    _arguments '1:vault:_vaulter_vaults'
                    ;;
                get|unset)
                    _arguments \
                        '1:key:_vaulter_keys' \
                        '--vault[Target vault]:vault:_vaulter_vaults'
                    ;;
                set|import)
                    _arguments '--vault[Target vault]:vault:_vaulter_vaults'
                    ;;
                export)
                    _arguments '*--vault[Vault to export]:vault:_vaulter_vaults'
                    ;;
                run)
                    _arguments \
                        '1:keyword:(with)' \
                        '2:vault:_vaulter_vaults'
                    ;;
                completions)
                    _arguments '1:shell:(zsh bash fish powershell elvish)'
                    ;;
            esac
            ;;
    esac
}

_vaulter_commands() {
    local -a commands
    commands=(
        'init:Initialize vaulter'
        'debug:Show debug information'
        'info:Show debug information (alias for debug)'
        'create:Create a new vault'
        'list:List all vaults'
        'ls:List all vaults (alias)'
        'delete:Delete a vault'
        'rm:Delete a vault (alias)'
        'use:Export a vault'\''s variables (no DB change)'
        'select:Export a vault'\''s variables (alias for use)'
        'switch:Switch to a vault (DB + export)'
        'sw:Switch to a vault (alias for switch)'
        'set:Set environment variables'
        'get:Get a variable value'
        'show:Show all variables in a vault'
        'print:Show all variables (alias for show)'
        'unset:Remove a variable'
        'export:Export variables as shell statements'
        'import:Import from a .env file'
        'run:Run a command with vault env injected'
        'completions:Print shell completion script'
    )
    _describe 'command' commands
}

_vaulter_vaults() {
    local -a vaults
    vaults=(${(f)"$(command vaulter _complete vaults 2>/dev/null)"})
    _describe 'vault' vaults
}

_vaulter_keys() {
    local -a keys
    local vault i
    # Look for --vault in the command line
    for ((i=1; i <= ${#words}; i++)); do
        if [[ "$words[$i]" == "--vault" ]]; then
            vault="$words[$((i+1))]"
            break
        fi
    done
    if [[ -n "$vault" ]]; then
        keys=(${(f)"$(command vaulter _complete vars --vault $vault 2>/dev/null)"})
    else
        keys=(${(f)"$(command vaulter _complete vars 2>/dev/null)"})
    fi
    _describe 'key' keys
}

_vaulter "$@"
