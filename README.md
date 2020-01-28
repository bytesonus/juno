# Project Gotham

## A minimalistic, unopinionated, microservices framework

__Q:__ What is it?  
__A:__ It's a framework that lets you connect modules to each other through a socket and expose generic API endpoints that can be reused. Modules can call each other's APIs along with parameters.

__Q:__ Where would I use it?  
__A:__ Well, as the name says, its primary function is to be used as a microservices framework. That being said, it's light enough to be used for IoT systems as well (consumes approx 200 KB RAM while running, with roughly 80-100 KB per connected module). IoT modules can expose APIs and call them from other modules. Any form of communication between modules can use this.  

__Q:__ Show me the code already!  
__A:__ Yeah, no. There's no code. The entire framework communicates using sockets. If you're looking for the format of communication, you can find it [here](./docs/communication-protocol.md)

__Q:__ Why would I replace my VM / Docker / Kubernetes with this?  
__A:__ You don't. You run this inside your orchestration service.

__Q:__ Why would I run this inside my orchestration service?  
__A:__ It's easier to explain with an example:

### Example 1:  

Let's say you have a server where you need to notify a user by email whenever their password changes. Your "database microservice" can simply have a hook (a hook is kinda like an event) that is fired when the password changes, and your "email microservice" can listen for that hook and send an email to the user automatically whenever their password changes. This lets you achieve data-binding as well as ensuring that your email notifications are always sent (just in case you forgot to add that subroutine in your code).

### Example 2:

Say you have multiple API endpoints that scale independently of each other like an "analysis microservice" (that does a lot of ML, for example) and a "auth microservice" (that simply authenticates users). Since they communicate with each other using sockets, on your local machine, they can all run together while on a production server, they can be isolated based on their requirement. The analysis module, for example, can be running on a system with a lot of GPU horsepower and the auth module can be running on a general purpose system, and still communicate well with each other using sockets.

__Q:__ That's cool and all, but I can just modify my existing code to support events for password changes and stuff.  
__A:__ Yes, you can.

__Q:__ Then why would I use this?  
__A:__ The password change event was just an example. There are so many more use-cases that can be solved. It's up to you how you want to take advantage of this framework. Also, since the modules are separated in a microservices fashion, it lets you scale up much more easily. When your load increases, your microservices can scale up much more easily, in a horizontal fashion, __without having to undergo a rewrite of your code.__

__Q:__ K. Anything else?  
__A:__ Although not initially intended, an added benefit of using this is that different parts of your code can be written in different languages and still work with each other. This makes it easy to interface across different frameworks. If your ML algorithm works best with Python and your API works best with NodeJS, they can still work with each other.

## Project status

Version 1.0 is pretty much done. It lets you declare functions and lets you call functions declared by other modules. It also lets you listen for a hook on other modules and trigger hooks as well. The plan is to make an ecosystem of very useful modules around this such that everything becomes easier to use, similar to NodeJS, where NodeJS in itself is not a big deal but thanks to NPM, everything becomes super easy to use.