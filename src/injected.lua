love.filesystem.setIdentity("_tmp_boxedmino");
love.filesystem.__boxedmino_masked_identity = "love";
love.filesystem.getIdentity = function()
    return love.filesystem.__boxedmino_masked_identity;
end
love.filesystem.setIdentity = function(identity)
    love.filesystem.__boxedmino_masked_identity = identity;
end