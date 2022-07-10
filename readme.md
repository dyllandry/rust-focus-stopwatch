# Rust Focus Stopwatch

This is a type of stopwatch meant to help track how long is spent working vs resting throughout a workday.

The stopwatch will specifically track:

- The total time spent focusing
- The total time spent resting
- The time spent in the current session, whether focus or rest.

# How it works

The stopwatch will start paused.

You can put the stopwatch into **Focus** or **Rest** mode.

Use **Focus** mode when you want to track how long you are working.

Use **Rest** mode when you want to track how long you are taking a break for.

The total time you spend in each mode will be tracked.

Changing modes or pausing will reset the current duration, but not affect any total durations.

# Ideal use case

I want to try and hit 5 hours of focused work in a work day. But I also need to take some breaks inbetween.

I start to work and put the stopwatch into **Focus** mode. After some time, I see the stopwatch says the current focus session has been going on for 15 minutes.

I start to feel a little distracted, and want to take a break. I swap to **Rest** mode and get up to pour a glass of water and use the bathroom. I come back and see the current rest session is at 3 minutes.

I begin to work again and put the stopwatch in **Focus** mode. After only 1 minute I'm interrupted by something non-work related that doesn't quite qualify as _resting_. For this situation, the stopwatch can be **paused**. This will stop adding any time to either the focus or rest durations. It will also reset the current session back to 0, in order to represent a literal break in what would have been a continious rest or focus session.

I repeat this process of changing between **Focus** and **Rest** mode and intermittently **pausing** until the total focus duration shows 5 hours and the total rest duration shows 1.5 hours.
