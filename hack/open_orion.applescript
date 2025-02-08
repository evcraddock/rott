#!/usr/bin/env osascript
on run argv
    set theURL to item 1 of argv
    tell application "Orion"
        tell window 1
            set newTab to make new tab with properties {URL:theURL}
            set current tab to newTab
        end tell
        activate
    end tell
end run
