Currently the following needs to be completed:
 - Fix the implementation of Modifiers, SQLite's implementation is very weird here
 - Add support for multiple modifiers to work correctly, dependent on above
 - Add support for the other modifiers that don't just directly add or subtract time
 - Handle edge cases: Feb 29 in leap years, month-end dates

 - Improve abstraction between modifier.rs and time_value.rs they do sim things.
 - strftime and timediff need to be done

 - Wayyyy more comprehesive testing here, maybe against sqlite smhow
 