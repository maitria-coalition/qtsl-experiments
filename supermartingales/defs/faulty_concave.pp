i = 1;
error = 0;
while i < 10 do
    assert(error == 0);
    if i < 4 then 
        p ~ Bernoulli(0.999)
    else
        p ~ Bernoulli(0.99)
    fi;
    if p == 1 then
        q ~ Bernoulli(0.5);
        if q == 1 then
            i += 1
        fi
    else
        error = 1
    fi
od
