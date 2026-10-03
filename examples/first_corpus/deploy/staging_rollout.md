Deploy the release candidate to staging first and verify the rollout before
promoting to production. The staging cluster mirrors production capacity, so
the health endpoints must be green on staging after every rollout step.
