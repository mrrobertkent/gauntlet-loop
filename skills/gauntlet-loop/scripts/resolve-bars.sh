#!/usr/bin/env sh
# Resolve the directory holding this project's bars (specs, requirements,
# acceptance criteria) and list what is in it.
#
# Precedence, first hit wins:
#   1. $1                              explicit path passed by the caller
#   2. $CLAUDE_PLUGIN_OPTION_BARS_DIR  the plugin's bars_dir option
#   3. bars_dir= in the project config (see CONFIG below)
#
# Exit codes:
#   0  resolved; path on stdout, contents listed after it
#   3  nothing configured
#   4  configured but the path does not exist
#
# Never guesses. Never searches the project.

set -u

CONFIG_REL=".claude/gauntlet-loop.conf"
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$PWD}"
CONFIG="$PROJECT_DIR/$CONFIG_REL"

bars=""
origin=""

if [ "$#" -gt 0 ] && [ -n "${1:-}" ]; then
  bars="$1"
  origin="argument"
elif [ -n "${CLAUDE_PLUGIN_OPTION_BARS_DIR:-}" ]; then
  bars="$CLAUDE_PLUGIN_OPTION_BARS_DIR"
  origin="plugin option bars_dir"
elif [ -f "$CONFIG" ]; then
  bars=$(sed -n 's/^[[:space:]]*bars_dir[[:space:]]*=[[:space:]]*//p' "$CONFIG" \
         | sed -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//" \
         | head -n 1)
  origin="$CONFIG_REL"
fi

if [ -z "$bars" ]; then
  echo "UNSET"
  echo "No bars directory configured. Ask the user for the bar."
  echo "To set one, create $CONFIG_REL in the project with:"
  echo "  bars_dir = path/to/your/specs"
  exit 3
fi

case "$bars" in
  /*|?:*|\\\\*) ;;                 # absolute (POSIX, Windows drive, UNC)
  *) bars="$PROJECT_DIR/$bars" ;;  # relative to the project
esac

if [ ! -d "$bars" ]; then
  echo "MISSING"
  echo "Configured via $origin, but no directory exists at:"
  echo "  $bars"
  echo "Tell the user the path is wrong. Do not go looking for another one."
  exit 4
fi

echo "$bars"
echo "(from $origin)"
echo
echo "Candidate bars:"
find "$bars" -type f \( -name '*.md' -o -name '*.markdown' -o -name '*.txt' \) \
  | sort \
  | sed 's/^/  /'
