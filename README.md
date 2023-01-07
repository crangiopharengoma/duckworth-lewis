# Duckworth Lewis Calculator

This is a simple rust lib that allows for target scores to be calculated using the Duckworth Lewis Standard Edition method.  Currently the Professional Edition and Duckworth-Lewis-Stern methodologies aren't published (anywhere that I'm aware of) so I can't implement them here. Note that international cricket uses the Duckworth-Lewis-Stern method so the results from this lib won't match what you see on TV.

## Features

This crate includes an optional CLI that can be used to play with the calculator. Include the feature 'cli' when building/running if you want to try it out. Use dlc help to get more information about how to use the cli.

The feature 'ser' allows for de/serialization (using serde) of the various structs and enums. This is required by the cli feature but can be separately enabled if you wanted it.

## CLI Usage

The below sequence of commands shows the necessary steps to capture the following scenario:
* A 50 over match being played between two ICC Full Members
* The match is disrupted after 12 overs have been completed in the first innings
* The side batting first has lost 1 wicket at the point the match is interrupted
* The match resumes with 10 overs removed from the team batting first's allocation (i.e. it is now a 40-overs-a-side match)
* The side batting first scores a total of 250 runs (total wickets lost are irrelevant) in the innings

```
dlc new 50 icc-full-member
dlc int 1 38 10 first
dlc target 250
```