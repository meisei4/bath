```


ok so that worked, but its wigglying everything a ton more than i was expecting when just looking at the top layer. Its
much easiier to recognize the warping artifacts that i assume are coming from the interpolation thing? but does this
mean we somehow moved the projection logic all to the vertex shader? If thats true can we also somehow move all the
transforms that are happening in the samplePerlinNoise function to affine transforms that happen in the vertex shader
and give us the same visual behavior as what we have now, now that we have moved this sort of like "projection"
behavior (including its interpolation maddness, idk i might be wrong about the reason for why its wiggling all over the
place as the noise texture gets sampled and scrolled and stuff.

Additional Thoughts:
You mentioned we’d only get one sample per vertex, and that we’d need to subdivide or use a geometry shader for better
results. But before I go that route:

    Can you provide a minimal version of what a per-vertex depth march might look like in the vertex shader? (I would prefer to save

Now that projection is driven from the vertex level, is there anything meaningful left to be done in the frag shader
beyond simple shading (like color lookup or lighting)?

You mentioned gl_Position, mvp, and others — I realize I’m already using mvp for the full-screen quad, but I hadn’t
really considered what more I could do with vertexNormal or vertexColor. I like the debugging ideabut i have no idea how
we could effectively use it in our current study of the ice sheets unless it could provide a way to visualize
interpolation error intuitively? (please ignore this if its not a game engineering standard usage)

But for now, I’m trying to really test the limits of what can be pushed into the vertex stage before resorting to
CPU-side logic or full mesh subdivision/tessalation