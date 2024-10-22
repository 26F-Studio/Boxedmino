love.filesystem.setIdentity("_tmp_boxedmino");
print("Running under sandboxed environment");
do
    local identity = "love";
    love.filesystem.setIdentity = function(new_identity)
        identity = new_identity;
    end
    love.filesystem.getIdentity = function()
        return identity;
    end
end