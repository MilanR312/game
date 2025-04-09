Small bevy crash course

bevy uses an ECS (Entity Component System) which is simular to a database

every entity can be represented as a row in the database

in each row we can then add multiple columns with each one a component
command.spawn((
    A,
    B,
    C // these are the components, the entity/row is automatically created
))

systems are regular functions we register to run at specific times like Startup or Update (once per frame)

Res and Resmut allow access to global resources in a non mutable or mutable way, like the global resource for all meshes

Bevy allows plugins to easily abstract parts of the app away and use other peoples code. This code doesnt need to be copy pasted and can be added with a single line of code.