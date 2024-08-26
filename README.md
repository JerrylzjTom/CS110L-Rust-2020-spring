# CS110L-2020-Spring
This is the repository for recording my [CS110L-2020-Spring](https://reberhardt.com/cs110l/spring-2020/) homework.
## Course Information
### Course Name: Safety in Systems Programming
### Course Number: CS110L
### Study Period: 2024.8.22-2024.8.26

## Environment
+ OS: Ubuntu 22.04
+ IDE: Visual Studio Code
+ rustc: 1.80.0

## Project
### Project 1 (The DEET Debugger):
Implement the DEET debugger (Dodgy Eliminator of Errors and Tragedies) to get the deets on those pesky bugs in your code.

**There is a bug that the debugger cannot show the file name and line number on Ubuntu 22.04, but it works on Ubuntu 20.04**, I have no idea why.
#### Implamentation
+ Stopping, resuming, and restarting the inferior
+ Printing a backtrace
+ Print stopped location
+ Setting breakpoints
+ Continuing from breakpoints
#### Extensions
+ Next line: like GDB `next` command
+ Print source code on stop
+ Print variables

### Project 2 (Balancebeam):
Load balancers are a crucial component for providing scalability and availability to networked services, and in this assignment, you’ll feel out the internals of a load balancer and learn what makes them tick!
#### Implamentation
+ Add multithreading
+ Use asynchronous I/O
+ Failover with passive health checks
+ Failover with active health checks
+ Rate limiting (fix window algorithm) [Article](https://konghq.com/blog/engineering/how-to-design-a-scalable-rate-limiting-algorithm)
