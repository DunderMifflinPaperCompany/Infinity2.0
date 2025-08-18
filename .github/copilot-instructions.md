For code reviews of Dependabot Security Updates, investigate the dependabot alert to understand the scope of the impact.  

If security advisory has details about a specific vulnerable function call, evaluate the codebase and search to see if the function is used.  

- If the function is used, indicate this in the review as a critical item that should be fixed.

- If the is not used, then indicate to in the review that no detected usage of the vulnerable function. Name the function, and give further instructions to search through transitive dependencies for additional possible downstream impact.
