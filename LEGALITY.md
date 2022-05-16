# Legality

This is not a legal document, and I am not a lawyer. I am _extremely_ displeased with how [Rockstar has interacted with the re3 project](https://www.courtlistener.com/docket/60335180/take-two-interactive-software-inc-v-papenhoff/), and this is one of the ways I am expressing that displeasure.

This codebase does not use any code from the original games or re3 directly. It uses public documentation from sources such as the [GTAMods wiki](https://gtamods.com/wiki/Main_Page), as the first game in the series came out _over two decades ago_ and has thus been well-studied. (There are people reading this document who are younger than these games. Isn't it fantastic that these intellectual properties are hoarded forever?)

With that being said, I do not intend to ignore re3's significant contribution to the study and development of these games. The people behind the re3 project put a significant amount of effort into reverse-engineering and rebuilding the games because they love them and want to see them thrive as the best versions of themselves and, not, say, [unfinished and poorly tested remasters](https://en.wikipedia.org/wiki/Grand_Theft_Auto:_The_Trilogy_%E2%80%93_The_Definitive_Edition#Reception).

---

To this end, I am instituting a modular [Chinese wall approach](https://en.wikipedia.org/wiki/Chinese_wall#Reverse_engineering) to using re3. You are *not* allowed to copy code from re3 directly (not that you could, given the different programming language), or to otherwise source logic from it directly. However, there are aspects of the game that cannot be reproduced without looking at the original game's code (that is to say, not reproducible through observation of game behaviour alone).

For the purpose of explanation, I'll take vehicle physics as an example. There is no way to reproduce this through observation alone. The vehicle handling in the games is highly dynamic and dependent on many internal and external state variables, which makes it infeasible to try to intuit how any given vehicle should behave in response to an input.

Instead, what you can do is:

1. Create an issue to indicate that you are about to study vehicle physics. This is to make it clear you are taking reverse-engineering ownership of this area.
2. Look at re3's source code and determine how vehicle physics works. Attempt to limit your exposure to the rest of the codebase as much as possible.
3. Write documentation and/or a specification of the physics work in `docs/game`.
4. **Do not write the vehicle physics code yourself**. Instead, another programmer _must_ write the code based on the documentation you have now produced based on your study.
5. The second programmer then submits their code. 

After this point:

- you are not allowed to work on that area of that code in any regard
- the second programmer is not allowed to study that area of re3

The intent of this approach is to allow for us to utilise the work that re3 has done in the most efficient way possible, without directly connecting "having looked at the game's code for a specific feature F" and "writing code for that feature F" for any one programmer.

That is to say, given two programmers X and Y, any feature F in the re3 source code can be reimplemented as long as X documents and Y implements, or vice-versa. The problems only arise if X and Y are the same individual.

If you think that this is stupid, I agree. There is no functional difference in the end between either of these approaches:
- studying the code directly, then reimplementing based on the study
- studying the code, producing documentation, and then having someone reimplement based on that documentation

but US case law says that the latter is mostly-legal and the former is mostly-not, so that's what we'll do.