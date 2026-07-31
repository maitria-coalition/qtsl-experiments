error = 0;
i = 0;
while i < 10 do
    assert(error == 0);
    p ~ Bernoulli(0.999);
    if p == 1 then
        q ~ Bernoulli(0.5);
        if q == 1 then
            i += 1
        fi
    else
        error = 1
    fi
od
