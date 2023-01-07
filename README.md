# Duckworth Lewis Calculator

This is a simple rust lib that allows for target scores to be calculated using the Duckworth Lewis Standard Edition method.  Currently the Professional Edition and Duckworth-Lewis-Stern methodologies aren't published (anywhere that I'm aware of) so I can't implement them here. Note that international cricket uses the Duckworth-Lewis-Stern method so the results from this lib won't match what you see on TV.

## Features

This crate includes an optional CLI that can be used to play with the calculator. Include the feature 'cli' when building/running if you want to try it out. Use dlc help to get more information about how to use the cli.

The feature 'ser' allows for de/serialization (using serde) of the various structs and enums. This is required by the cli feature but can be separately enabled if you wanted it.